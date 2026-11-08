//! `usable_windows` against hand-worked counts (claude-321).

use reply_cap::{usable_windows, MIN_SCORED};

/// Two stray positions before the first BOS, then three documents (BOS = 0).
/// - Doc A, 6 tokens: targets p 1..=5 with mask 1 1 0 1 1. With context 3, the chunks are 1..4
///   (2 scored) and 4..6 (2 scored).
/// - Doc B, 5 tokens: targets p 1..=4 with mask 0 0 0 1. The chunks are 1..4 (0 scored, so not a
///   window) and 4..5 (1 scored).
/// - Doc C: a lone BOS, so no targets.
fn fixture() -> (Vec<u16>, Vec<u8>) {
    let tokens = vec![9, 9, 0, 5, 6, 7, 8, 9, 0, 7, 7, 7, 7, 0];
    let mask = vec![1, 1, 0, 1, 1, 0, 1, 1, 1, 0, 0, 0, 1, 0];
    (tokens, mask)
}

#[test]
fn chunks_count_by_hand() {
    let (t, m) = fixture();
    // Three documents; windows A1, A2, B2; usable (at least 2 scored) A1, A2.
    assert_eq!(usable_windows(&t, &m, 0, 3, 2), (3, 3, 2));
    // At least 1 scored: every window is usable.
    assert_eq!(usable_windows(&t, &m, 0, 3, 1), (3, 3, 3));
    // Context 5: Doc A is one chunk (4 scored); Doc B one chunk (1 scored).
    assert_eq!(usable_windows(&t, &m, 0, 5, 2), (3, 2, 1));
}

#[test]
fn a_masked_bos_target_is_never_scored() {
    // A BOS position with mask 1 starts a document; it is never a scored target.
    let t = vec![0, 4, 4, 0, 4];
    let m = vec![1, 1, 1, 1, 1];
    // Docs [0,4,4] and [0,4]; context 2: chunk 1..3 (2 scored), chunk 4..5 (1 scored).
    assert_eq!(usable_windows(&t, &m, 0, 2, 2), (2, 2, 1));
}

#[test]
fn the_trainer_filter_is_four() {
    assert_eq!(MIN_SCORED, 4);
}
