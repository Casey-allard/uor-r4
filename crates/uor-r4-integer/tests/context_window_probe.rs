//! Probe: the bounded ring and settlement gate for the `Context` stage.
//!
//! This is the transferable part of CIRISKernel: a span may leave a bounded ring only when it is
//! **settled**, and the stub it leaves behind must keep exactly what the declared read consumes.
//! Two reads are compared against full history across several wraps:
//!
//! - a fold read (a commutative sum), which a sum stub preserves exactly;
//! - a selection read (top-k), which a sum stub does **not** preserve, and which a mergeable
//!   bounded top-k sketch does.
//!
//! The witness pair shows why an unsettled overwrite must be refused: two histories the stub cannot
//! distinguish, which a later read sends to different answers.
//!
//! `cargo test -p uor-r4-integer --test context_window_probe`

/// One span: an id and the value it contributes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Span {
    id: u64,
    value: i64,
}

fn better(a: &Span, b: &Span) -> bool {
    a.value > b.value || (a.value == b.value && a.id < b.id)
}

/// A mergeable bounded top-k: the largest `k` values, ties to the smaller id.
#[derive(Clone, PartialEq, Eq, Debug)]
struct TopK {
    k: usize,
    entries: Vec<Span>,
}

impl TopK {
    fn new(k: usize) -> Self {
        TopK {
            k,
            entries: Vec::new(),
        }
    }

    fn insert(&mut self, span: Span) {
        if self.k == 0 {
            return;
        }
        self.entries.push(span);
        self.entries
            .sort_by(|a, b| match (better(a, b), better(b, a)) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            });
        self.entries.truncate(self.k);
    }

    /// Associative and commutative: insertion into a bounded top-k is a join.
    fn merge(&self, other: &TopK) -> TopK {
        let mut out = self.clone();
        for span in &other.entries {
            out.insert(*span);
        }
        out
    }

    fn ids(&self) -> Vec<u64> {
        self.entries.iter().map(|span| span.id).collect()
    }
}

/// The bounded ring: fixed slots, a cursor, and a stub per dropped span.
struct Ring {
    slots: Vec<Option<Span>>,
    settled: Vec<bool>,
    cursor: usize,
    folded_sum: i128,
    folded_top: TopK,
    /// Audit only: the real ring does not keep dropped ids, and this vector is the one unbounded
    /// part of the probe.
    dropped: Vec<u64>,
}

impl Ring {
    fn new(capacity: usize, sketch: usize) -> Self {
        assert!(capacity > 0);
        Ring {
            slots: vec![None; capacity],
            settled: vec![false; capacity],
            cursor: 0,
            folded_sum: 0,
            folded_top: TopK::new(sketch),
            dropped: Vec::new(),
        }
    }

    /// Write `span` into the next slot. When the slot holds an **unsettled** span, refuse and
    /// return its id; the ring is unchanged. A settled span is folded into the stub and dropped.
    fn push(&mut self, span: Span, settled: bool) -> Result<(), u64> {
        let slot = self.cursor % self.slots.len();
        if let Some(old) = self.slots[slot] {
            if !self.settled[slot] {
                return Err(old.id);
            }
            self.folded_sum += i128::from(old.value);
            self.folded_top.insert(old);
            self.dropped.push(old.id);
        }
        self.slots[slot] = Some(span);
        self.settled[slot] = settled;
        self.cursor += 1;
        Ok(())
    }

    fn live_sum(&self) -> i128 {
        self.slots
            .iter()
            .flatten()
            .map(|span| i128::from(span.value))
            .sum()
    }

    fn sum(&self) -> i128 {
        self.folded_sum + self.live_sum()
    }

    /// The top-k over every span the ring has seen: the fuse of the stub and the live slots. The
    /// sketch capacity must be at least `k`, or the dropped candidates cannot answer the read.
    fn top(&self, k: usize) -> TopK {
        let mut out = TopK::new(k);
        for span in self.folded_top.entries.iter() {
            out.insert(*span);
        }
        for span in self.slots.iter().flatten() {
            out.insert(*span);
        }
        out
    }

    fn live_len(&self) -> usize {
        self.slots.iter().flatten().count()
    }
}

/// The full-history record the ring is compared against.
struct History {
    spans: Vec<Span>,
}

impl History {
    fn new() -> Self {
        History { spans: Vec::new() }
    }

    fn push(&mut self, span: Span) {
        self.spans.push(span);
    }

    fn sum(&self) -> i128 {
        self.spans.iter().map(|span| i128::from(span.value)).sum()
    }

    fn top(&self, k: usize) -> TopK {
        let mut out = TopK::new(k);
        for span in &self.spans {
            out.insert(*span);
        }
        out
    }
}

#[test]
fn the_fold_and_the_sketch_survive_every_settled_wrap() {
    let capacity = 8;
    let sketch = 4;
    let mut ring = Ring::new(capacity, sketch);
    let mut history = History::new();

    for step in 0..40u64 {
        // A deterministic, sign-changing stream with repeated values.
        let value = ((step * 37) % 23) as i64 - 11;
        let span = Span { id: step, value };
        history.push(span);
        ring.push(span, true).expect("every span is settled");
        assert_eq!(ring.sum(), history.sum(), "fold read at step {step}");
        assert_eq!(
            ring.top(3).ids(),
            history.top(3).ids(),
            "selection read at step {step}"
        );
        assert!(ring.live_len() <= capacity, "the ring stayed bounded");
    }

    assert!(ring.dropped.len() >= 3 * capacity, "at least three wraps");
    assert_eq!(ring.dropped.len(), 40 - capacity);
}

#[test]
fn an_unsettled_push_is_refused_and_a_sum_stub_has_a_witness_pair() {
    let mut ring = Ring::new(2, 1);
    ring.push(Span { id: 1, value: 3 }, true).expect("settled");
    ring.push(Span { id: 2, value: -1 }, false).expect("live");
    // The third push reuses slot 0 (span 1, settled); the fourth reaches slot 1, where the
    // unsettled span 2 sits.
    ring.push(Span { id: 3, value: 5 }, true)
        .expect("slot 0 is settled");
    let before = (ring.sum(), ring.top(1).ids());
    let refused = ring.push(Span { id: 4, value: 9 }, true);
    assert_eq!(refused, Err(2), "the unsettled span 2 blocks the slot");
    assert_eq!((ring.sum(), ring.top(1).ids()), before, "state unchanged");

    // The witness pair for a sum-only stub. Two histories drop different spans whose sums are
    // equal, into a live span of the same value; the sum-only stub is identical for the declared
    // value read, yet a later top-1 read differs. That is exactly the pair an unsettled drop would
    // make indistinguishable.
    let mut left = Ring::new(1, 0); // a sum-only stub keeps no candidates
    let mut right = Ring::new(1, 0);
    for (id, value) in [(10u64, 3i64), (11, -1), (12, 0)] {
        left.push(Span { id, value }, true).expect("left");
    }
    for (id, value) in [(20u64, 1i64), (21, 1), (22, 0)] {
        right.push(Span { id, value }, true).expect("right");
    }
    assert_eq!(left.sum(), right.sum(), "the two sum stubs are identical");
    assert_eq!(left.live_len(), right.live_len());

    // The full histories have the same sum and different top-1, so a sum-only stub cannot answer.
    let left_history = [3i64, -1, 0];
    let right_history = [1i64, 1, 0];
    assert_eq!(
        left_history.iter().sum::<i64>(),
        right_history.iter().sum::<i64>()
    );
    assert_ne!(
        *left_history.iter().max().expect("nonempty"),
        *right_history.iter().max().expect("nonempty")
    );

    // A ring that keeps one candidate distinguishes them, and matches full history.
    let mut sketched = Ring::new(1, 1);
    for (id, value) in [(10u64, 3i64), (11, -1), (12, 0)] {
        sketched.push(Span { id, value }, true).expect("sketched");
    }
    assert_eq!(
        sketched.top(1).entries.first().map(|span| span.value),
        Some(3),
        "the sketch keeps the dropped maximum"
    );

    eprintln!(
        "witness: histories {:?} and {:?} share the sum {} and the live value, but their top-1 \
         differ ({} vs {}); a sum-only stub cannot answer, which is why the unsettled overwrite \
         is refused (blocked id {})",
        left_history,
        right_history,
        left_history.iter().sum::<i64>(),
        left_history.iter().max().expect("nonempty"),
        right_history.iter().max().expect("nonempty"),
        refused.expect_err("blocked")
    );
}

#[test]
fn the_stub_merge_is_associative_and_commutative() {
    let spans = [
        Span { id: 1, value: 7 },
        Span { id: 2, value: -3 },
        Span { id: 3, value: 7 },
        Span { id: 4, value: 0 },
        Span { id: 5, value: 11 },
    ];
    let mut a = TopK::new(3);
    let mut b = TopK::new(3);
    let mut c = TopK::new(3);
    for span in &spans[..2] {
        a.insert(*span);
    }
    for span in &spans[2..4] {
        b.insert(*span);
    }
    for span in &spans[4..] {
        c.insert(*span);
    }
    let left = a.merge(&b).merge(&c);
    let right = a.merge(&b.merge(&c));
    assert_eq!(left, right, "merge is associative");
    assert_eq!(a.merge(&b), b.merge(&a), "merge is commutative");
    assert_eq!(
        left.ids(),
        History {
            spans: spans.to_vec()
        }
        .top(3)
        .ids(),
        "merge equals the full-history top-k"
    );
}
