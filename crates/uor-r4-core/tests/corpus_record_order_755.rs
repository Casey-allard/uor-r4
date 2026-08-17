//! Regression coverage for #755: `compiler::load_corpus_bytes` must
//! reconstruct the same per-story `input`/context sequence regardless of
//! the physical on-disk order of `corpus.records`.
//!
//! #752 found that the content-addressed sharded observation pipeline
//! routes records by a hash of local context, not by story or position, so
//! a corpus's on-disk record order need not be per-story-contiguous. Before
//! #755's fix, `load_corpus_bytes` derived `input[i]` (and
//! `runtime::history_token`'s deeper context) from physical array
//! adjacency (`story[i-1] == story[i]`), which silently collapsed to a BOS
//! reset almost everywhere once records were interleaved -- turning clean
//! source text into word salad at compile time even though the corpus text
//! itself was never wrong.
//!
//! This test builds one small corpus's v4 records in natural per-story
//! order and the exact same records in a deliberately interleaved
//! (round-robin-across-stories) order, and asserts `load_corpus_bytes`
//! reconstructs identical `(story, span_start, input, next)` tuples from
//! both -- i.e. that on-disk order no longer changes what the compiler
//! believes each story's sequence was.

use uor_r4_core::transformerless::compiler;

const STORIES: u32 = 3;
const TOKENS_PER_STORY: u32 = 4;

fn encode_meta(n: u64, stories: u64) -> [u8; 25] {
    let mut meta = [0u8; 25];
    meta[0..8].copy_from_slice(&n.to_le_bytes());
    meta[8..16].copy_from_slice(&stories.to_le_bytes());
    meta[16..24].copy_from_slice(&0u64.to_le_bytes()); // rng, unused by this test
    meta[24] = 1; // done
    meta
}

/// One fabricated record: token id is `100*story + position`, so every
/// record's `next` value is trivially traceable back to its intended
/// `(story, position)` regardless of where it ends up on disk.
fn encode_record(story: u32, position: u32) -> [u8; 88] {
    let next = 100 * story + position;
    let mut top_tokens = [0u32; 8];
    let mut top_weights = [0u32; 8];
    top_tokens[0] = next;
    top_weights[0] = 1;
    compiler::encode_v4_record(
        story,
        next,
        &top_tokens,
        &top_weights,
        (position, position + 1),
        (u32::MAX, u32::MAX),
    )
}

/// All `(story, position)` pairs for the fixture corpus, in natural
/// per-story-contiguous order.
fn natural_order_pairs() -> Vec<(u32, u32)> {
    (0..STORIES)
        .flat_map(|story| (0..TOKENS_PER_STORY).map(move |position| (story, position)))
        .collect()
}

/// The same pairs, interleaved round-robin across stories -- every
/// consecutive pair in this order crosses a story boundary, the same
/// pattern #752 measured directly (99.93% of consecutive records) on a
/// real bundle produced by the sharded observation pipeline.
fn shuffled_order_pairs() -> Vec<(u32, u32)> {
    (0..TOKENS_PER_STORY)
        .flat_map(|position| (0..STORIES).map(move |story| (story, position)))
        .collect()
}

fn records_bytes(pairs: &[(u32, u32)]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(pairs.len() * 88);
    for &(story, position) in pairs {
        bytes.extend_from_slice(&encode_record(story, position));
    }
    bytes
}

/// `(story, span_start, input, next)` tuples, sorted, so two parses that
/// reconstruct the same semantic content compare equal regardless of the
/// order `Corpus`'s own arrays happen to hold them in.
fn canonical_tuples(corpus: &compiler::Corpus) -> Vec<(u32, u32, u32, u32)> {
    let mut tuples: Vec<(u32, u32, u32, u32)> = (0..corpus.n)
        .map(|i| {
            (
                corpus.story[i],
                corpus.span_start[i],
                corpus.input[i],
                corpus.next[i],
            )
        })
        .collect();
    tuples.sort();
    tuples
}

#[test]
fn load_corpus_bytes_reconstructs_the_same_context_regardless_of_on_disk_order() {
    let n = (STORIES * TOKENS_PER_STORY) as u64;
    let meta = encode_meta(n, STORIES as u64);

    let natural = records_bytes(&natural_order_pairs());
    let shuffled = records_bytes(&shuffled_order_pairs());
    assert_ne!(
        natural, shuffled,
        "the fixture must actually exercise different on-disk byte orders"
    );

    let natural_corpus =
        compiler::load_corpus_bytes(&meta, &natural, None).expect("natural-order corpus parses");
    let shuffled_corpus =
        compiler::load_corpus_bytes(&meta, &shuffled, None).expect("shuffled-order corpus parses");

    assert_eq!(
        canonical_tuples(&natural_corpus),
        canonical_tuples(&shuffled_corpus),
        "on-disk record order must not change the reconstructed context"
    );

    // Spot-check the actual chaining semantics on the shuffled parse
    // directly: story 1's position-2 record must see story 1's position-1
    // next token as its input, and position 0 of every story must see BOS
    // (1) as its input -- this is the concrete property #755 restores.
    let story1_pos1_next = 100u32 + 1;
    let mut checked_position_1 = false;
    let mut checked_position_0 = false;
    for i in 0..shuffled_corpus.n {
        if shuffled_corpus.story[i] == 1 && shuffled_corpus.span_start[i] == 2 {
            assert_eq!(
                shuffled_corpus.input[i], story1_pos1_next,
                "story 1 position 2 must chain from story 1 position 1's next token"
            );
            checked_position_1 = true;
        }
        if shuffled_corpus.span_start[i] == 0 {
            assert_eq!(
                shuffled_corpus.input[i], 1,
                "the first position of every story must see BOS regardless of on-disk order"
            );
            checked_position_0 = true;
        }
    }
    assert!(
        checked_position_1,
        "fixture must contain story=1, span_start=2"
    );
    assert!(
        checked_position_0,
        "fixture must contain at least one span_start=0 record"
    );
}
