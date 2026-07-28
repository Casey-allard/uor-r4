//! Tests for the evaluation context builder (issue #237).
//!
//! The previous construction indexed the corpus with per-slot vector
//! rotation offsets (j·17 mod D) as time lags — stride-17 sampling, in
//! reversed order, across story boundaries, with a phantom trailing zero.
//! `eval_context` must instead produce the last WINDOW tokens, oldest →
//! newest, ending at `input[i]`, bounded to the position's own story.

#![cfg(not(target_arch = "wasm32"))]

use uor_r4_core::transformerless::compiler::{Corpus, WINDOW};
use uor_r4_graph_certify::certify::eval_context;

fn corpus(story: Vec<u32>, input: Vec<u32>) -> Corpus {
    let n = input.len();
    let stories = story.iter().copied().max().unwrap_or(0) as u64 + 1;
    Corpus {
        n,
        stories,
        story,
        input,
        next: vec![0; n],
        t_argmax: vec![0; n],
        top_tokens: vec![[0; 8]; n],
        top_weights: vec![[0; 8]; n],
        span_start: vec![0; n],
        span_end: vec![0; n],
        byte_start: vec![0; n],
        byte_end: vec![0; n],
        hidden: None,
    }
}

#[test]
fn last_window_tokens_chronological_ending_at_i() {
    // 12 tokens, one story; position 10 → tokens 3..=10 (WINDOW = 8).
    let input: Vec<u32> = (100..112).collect();
    let c = corpus(vec![0; 12], input);
    let hist = eval_context(&c, 10);
    assert_eq!(hist.len(), WINDOW);
    assert_eq!(hist, (103..=110).collect::<Vec<u32>>());
    assert_eq!(
        *hist.last().unwrap(),
        110,
        "most recent token must be input[i]"
    );
}

#[test]
fn story_boundary_truncates_history() {
    // Story 1 starts at index 4; position 6 must see only tokens 4..=6.
    let story = vec![0, 0, 0, 0, 1, 1, 1, 1];
    let input: Vec<u32> = (200..208).collect();
    let c = corpus(story, input);
    let hist = eval_context(&c, 6);
    assert_eq!(hist, vec![204, 205, 206]);
}

#[test]
fn position_zero_yields_single_token() {
    let c = corpus(vec![0; 4], vec![7, 8, 9, 10]);
    let hist = eval_context(&c, 0);
    assert_eq!(hist, vec![7]);
}

#[test]
fn no_zero_padding_and_no_reordering() {
    // Regression guard against the old construction's phantom trailing 0
    // and reversed order: with distinct tokens, output is strictly the
    // consecutive ascending slice.
    let input: Vec<u32> = (300..320).collect();
    let c = corpus(vec![0; 20], input.clone());
    for i in [3usize, 9, 19] {
        let hist = eval_context(&c, i);
        let lo = i + 1 - hist.len();
        assert_eq!(hist, input[lo..=i].to_vec());
        assert!(!hist.contains(&0));
    }
}
