//! Decode an observed teacher corpus (`corpus.meta` + `corpus.records`)
//! back into readable text through a bundle's own tokenizer, using the
//! exact same parsing the compiler uses
//! ([`uor_r4_core::transformerless::compiler::load_corpus_bytes`]) rather
//! than reimplementing the record layout by hand.
//!
//! This exists to inspect what a compiled bundle's `tless_store.bin` count
//! tables were actually trained against (#752, continuing #745).
//!
//! IMPORTANT: `load_corpus_bytes` auto-detects the record width (88/48/32
//! /12 bytes) from `meta.n` vs. the records file length, cascading in that
//! order (88 only if the file is an *exact* multiple of 88 *and* at least
//! `n*88` bytes; otherwise it falls through to 48, then 32, then legacy
//! 12). A tool that assumes one fixed width and just chunks the records
//! file at that stride can silently desynchronize and produce nonsense —
//! this happened once already while building this tool (see the #745/#752
//! discussion): a naive 88-byte assumption happened to divide this
//! particular bundle's `corpus.records` evenly too, produced plausible-
//! looking but actually-misaligned "decoded text", and was caught only by
//! checking record count against `meta.n` precisely. Always go through
//! `load_corpus_bytes` (or replicate its exact cascade) rather than
//! guessing from a file-size hex dump.
//!
//! `load_corpus_bytes` also refuses to parse unless `meta[24] == 1`
//! ("done"). For a bundle whose checked-in `corpus.meta` shows `done=0`
//! (an interrupted/incomplete generation, as observed for at least one
//! local bundle), this tool deliberately overrides that one byte to `1`
//! before parsing so the corpus can still be inspected — the record count
//! actually used is still exactly meta.n as committed, which
//! load_corpus_bytes truncates the records buffer to; nothing about the
//! record-layout/record-count logic itself is bypassed, only the
//! "generation still in flight" refusal.
//!
//! Usage:
//!   cargo run -q -p uor-r4-core --example decode_corpus -- \
//!     <corpus.meta> <corpus.records> <tokenizer.bin> [--stories N] [--all]
//!
//! Prints, in order:
//!   1. The parsed `corpus.meta` header and which record width
//!      `load_corpus_bytes` selected for this pair, plus whether every
//!      story's records are physically contiguous in the file (the
//!      assumption `load_corpus_bytes` itself relies on when deriving
//!      `input[i] = next[i-1]` per story).
//!   2. Up to N decoded stories (default 5; `--all` decodes every story
//!      present).
//!   3. A frequency count of the most common `next` tokens across the
//!      corpus, decoded to their token text.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use uor_r4_core::transformerless::compiler;
use uor_r4_core::transformerless::scenarios::Tokenizer;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!(
            "usage: decode_corpus <corpus.meta> <corpus.records> <tokenizer.bin> [--stories N] [--all]"
        );
        std::process::exit(2);
    }
    let meta_path = &args[1];
    let records_path = &args[2];
    let tokenizer_path = &args[3];

    let mut story_limit: usize = 5;
    let mut show_all = false;
    let mut i = 4;
    while i < args.len() {
        match args[i].as_str() {
            "--stories" => {
                i += 1;
                story_limit = args
                    .get(i)
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(story_limit);
            }
            "--all" => show_all = true,
            other => eprintln!("ignoring unrecognized argument: {other}"),
        }
        i += 1;
    }

    let mut meta_bytes = fs::read(meta_path).expect("read corpus.meta");
    let records_bytes = fs::read(records_path).expect("read corpus.records");
    let tokenizer_bytes = fs::read(tokenizer_path).expect("read tokenizer.bin");
    let tokenizer = Tokenizer::from_bytes(&tokenizer_bytes).expect("parse tokenizer.bin");

    println!("=== corpus.meta ===");
    if meta_bytes.len() != 25 {
        println!(
            "corpus.meta is {} bytes, expected exactly 25; cannot parse",
            meta_bytes.len()
        );
        std::process::exit(1);
    }
    let n = u64::from_le_bytes(meta_bytes[0..8].try_into().unwrap());
    let stories_claimed = u64::from_le_bytes(meta_bytes[8..16].try_into().unwrap());
    let rng = u64::from_le_bytes(meta_bytes[16..24].try_into().unwrap());
    let done = meta_bytes[24];
    println!("n={n} stories={stories_claimed} rng={rng} done={done}");
    println!("corpus.records: {} bytes on disk", records_bytes.len());
    if done != 1 {
        println!(
            "NOTE: done={done} (not 1) - this generation was not marked complete. Forcing the \
             done byte to 1 in a local copy of the metadata so load_corpus_bytes will parse it \
             for inspection; the record count used is still exactly meta.n as committed."
        );
        meta_bytes[24] = 1;
    }

    let corpus = compiler::load_corpus_bytes(&meta_bytes, &records_bytes, None)
        .expect("load_corpus_bytes failed to parse this meta/records pair");
    println!(
        "load_corpus_bytes parsed {} records across {} stories (as declared in meta)",
        corpus.n, corpus.stories
    );

    // Verify the same physical-contiguity assumption load_corpus_bytes
    // relies on for its own input[i] = next[i-1] derivation: for each
    // story, are all of its record indices contiguous in the parsed
    // stream? If not, load_corpus_bytes's own reconstructed `input`
    // sequence has spurious BOS resets wherever an interleaving boundary
    // falls, which corrupts the context the compiler trains on.
    let mut story_indices: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    for (idx, &story_id) in corpus.story.iter().enumerate() {
        story_indices.entry(story_id).or_default().push(idx);
    }
    let mut noncontiguous = 0usize;
    let mut worst: Vec<(u32, usize, usize, usize)> = Vec::new();
    for (story_id, indices) in &story_indices {
        let lo = *indices.first().unwrap();
        let hi = *indices.last().unwrap();
        let span = hi - lo + 1;
        if span != indices.len() {
            noncontiguous += 1;
            worst.push((*story_id, indices.len(), lo, hi));
        }
    }

    worst.sort_by_key(|&(_, count, lo, hi)| std::cmp::Reverse((hi - lo + 1).saturating_sub(count)));
    println!(
        "distinct story ids present: {}, non-contiguous: {}",
        story_indices.len(),
        noncontiguous
    );
    if noncontiguous > 0 {
        println!("worst offenders (story, record_count, first_index, last_index):");
        for (story_id, count, lo, hi) in worst.iter().take(5) {
            println!("  story={story_id} count={count} first_idx={lo} last_idx={hi}");
        }
    }

    println!("\n=== decoded stories ===");
    let limit = if show_all {
        story_indices.len()
    } else {
        story_limit
    };
    for (story_id, indices) in story_indices.iter().take(limit) {
        let tokens: Vec<u32> = indices.iter().map(|&idx| corpus.next[idx]).collect();
        let text = tokenizer.decode(&tokens);
        println!(
            "--- story {story_id} ({} tokens, indices {}..={}) ---\n{}\n",
            tokens.len(),
            indices.first().unwrap(),
            indices.last().unwrap(),
            text
        );
    }

    println!(
        "=== next-token frequency across all {} records ===",
        corpus.n
    );
    let mut counts: BTreeMap<u32, u64> = BTreeMap::new();
    for &token in &corpus.next {
        *counts.entry(token).or_insert(0) += 1;
    }
    let mut by_count: Vec<(u32, u64)> = counts.into_iter().collect();
    by_count.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    println!("distinct next-token ids: {}", by_count.len());
    for (token_id, count) in by_count.iter().take(40) {
        let text = tokenizer.decode(std::slice::from_ref(token_id));
        let pct = 100.0 * (*count as f64) / (corpus.n as f64);
        println!("  id={token_id:<8} count={count:<8} ({pct:5.2}%)  {text:?}");
    }
}
