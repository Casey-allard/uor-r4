//! The product-key candidate join, checked against the float path's rule.
//!
//! The port's selection must reproduce `stack_memory.rs`'s `top` +
//! candidate-join exactly: per side the `top_k` best sub-keys by score
//! descending with ties to the lower index, then the `top_k` best pairs by
//! summed score with ties to the lower slot (`first * sub_keys + second`).
//! Weights and the value gather come later in the port; this test fixes the
//! ordering, which is where an off-by-one or a tie-break difference would
//! silently change which slots a served model reads.

use uor_r4_integer::stack::{stack_memory_select, MAX_SELECT_SIDE};

/// The float path's rule, written directly from `stack_memory.rs`.
fn reference(first: &[i32], second: &[i32], top_k: usize) -> Vec<(u16, u16)> {
    let sub_keys = first.len();
    let top = |scores: &[i32]| -> Vec<usize> {
        let mut order: Vec<usize> = (0..scores.len()).collect();
        order.sort_by(|&a, &b| scores[b].cmp(&scores[a]).then(a.cmp(&b)));
        order.truncate(top_k.min(scores.len()));
        order
    };
    let (firsts, seconds) = (top(first), top(second));
    let mut candidates: Vec<(i64, usize, usize)> = Vec::new();
    for &a in &firsts {
        for &b in &seconds {
            candidates.push((i64::from(first[a]) + i64::from(second[b]), a, b));
        }
    }
    candidates.sort_by(|x, y| {
        y.0.cmp(&x.0)
            .then((x.1 * sub_keys + x.2).cmp(&(y.1 * sub_keys + y.2)))
    });
    candidates.truncate(top_k);
    candidates
        .into_iter()
        .map(|(_, a, b)| (a as u16, b as u16))
        .collect()
}

fn check(first: &[i32], second: &[i32], top_k: usize) {
    let mut out = vec![(0u16, 0u16); top_k * top_k];
    let written = stack_memory_select(first, second, top_k, &mut out);
    let expected = reference(first, second, top_k);
    assert_eq!(
        &out[..written],
        expected.as_slice(),
        "selection differs for first={first:?} second={second:?} top_k={top_k}"
    );
}

#[test]
fn the_selection_matches_the_float_rule_on_distinct_scores() {
    check(&[5, -3, 9, 0, 7, -8, 2, 1], &[1, 4, -2, 8, 0, 6, -5, 3], 2);
    check(&[5, -3, 9, 0, 7, -8, 2, 1], &[1, 4, -2, 8, 0, 6, -5, 3], 3);
    check(&[0, -1, -2, -3], &[-3, -2, -1, 0], 1);
}

#[test]
fn the_selection_matches_the_float_rule_on_ties() {
    // Every score equal: the rule is the lower index, then the lower slot.
    check(&[7, 7, 7, 7], &[7, 7, 7, 7], 2);
    // Ties within a side and across pairs.
    check(&[3, 3, 1, 1], &[2, 2, 4, 4], 2);
    check(&[4, 4, 4], &[0, 0, 0], 3);
    // Negative scores with a tie at the top.
    check(&[-1, -1, -9], &[-1, -1, -9], 2);
}

#[test]
fn a_k_larger_than_the_sub_keys_keeps_what_exists_and_a_zero_k_writes_nothing() {
    let first = [2, 1, 0];
    let second = [0, 1, 2];
    // The float path truncates to the side length.
    check(&first, &second, 3);
    // `top_k = 0` and an undersized buffer write nothing.
    let mut out = [(9u16, 9u16); 4];
    assert_eq!(stack_memory_select(&first, &second, 0, &mut out), 0);
    assert_eq!(stack_memory_select(&first, &second, 2, &mut out[..3]), 0);
    assert_eq!(out[0], (9, 9), "an undersized buffer must not be written");
}

#[test]
fn the_kernel_refuses_a_side_beyond_its_stack_bound() {
    // The shape's own bound keeps the kernel's buffers finite; a larger side
    // returns nothing rather than reading past them.
    let big = vec![0i32; MAX_SELECT_SIDE + 1];
    let mut out = vec![(0u16, 0u16); 1];
    assert_eq!(stack_memory_select(&big, &big, 1, &mut out), 0);
}
