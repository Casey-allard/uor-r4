//! CUDA tests of the exact n-gram counter (claude-412): GPU records equal the
//! CPU oracle's byte for byte, the SHA does not move across partition budgets
//! or runs, and a consistent planted single-count change moves it. Run on wick's 4090
//! with `--release` (the ten-million-token oracle is slow in a debug build).

// The seeds are written `0x<unit>_<case>` (`0x412_1` is claude-412's first case), so their
// digit groups name the unit and the case rather than bytes.
#![allow(clippy::unusual_byte_groupings)]

use crate::ngram_count::{NgramAssembler, NgramContext, NgramOrderDigest, NgramTriple, NGRAM_BOS};
use crate::ngram_count_device::NgramCounter;
use crate::ngram_count_oracle::{oracle_adjusted, oracle_count, plant_one_count};

/// A deterministic stream: one in four ids uniform over 4,096, the rest over
/// `narrow` ids, so high orders repeat; `narrow = 0` gives all-uniform ids.
/// Every 61st id is BOS (0), a planted record start, so the device's
/// document-boundary rule (claude-415) meets BOS in every slot.
fn stream(len: usize, seed: u64, narrow: u32) -> Vec<u16> {
    let mut state = seed | 1;
    (0..len)
        .map(|i| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let r = (state >> 20) as u32;
            if i % 61 == 0 {
                return 0;
            }
            (if narrow == 0 || r.is_multiple_of(4) { r % 4096 } else { r % narrow }) as u16
        })
        .collect()
}

fn gpu(
    counter: &NgramCounter,
    order: u32,
    budget: u64,
) -> (NgramOrderDigest, Vec<NgramTriple>, Vec<crate::ngram_count::NgramContext>, usize) {
    let mut assembler = NgramAssembler::new(order, true).expect("assembler");
    let plan = counter.count_order(order, budget, &mut assembler).expect("count");
    let (digest, triples, contexts) = assembler.finish();
    (digest, triples, contexts, plan.len())
}

#[test]
fn ngram_count_cuda_matches_the_oracle() {
    for (len, seed, narrow) in [
        (100_000usize, 0x412_1u64, 0u32),
        (100_000, 0x412_2, 13),
        (1_000_000, 0x412_3, 13),
        (10_000_000, 0x412_4, 29),
    ] {
        let tokens = stream(len, seed, narrow);
        let counter = NgramCounter::new(0, &tokens).expect("counter");
        for order in 0..=8u32 {
            let (digest, triples, contexts, _) = gpu(&counter, order, u64::MAX);
            let (want, want_triples, want_contexts) = oracle_count(&tokens, order).expect("oracle");
            assert_eq!(digest, want, "digest, len {len} narrow {narrow} order {order}");
            assert!(triples == want_triples, "triples, len {len} order {order}");
            assert!(contexts == want_contexts, "contexts, len {len} order {order}");
            eprintln!(
                "match len {len} narrow {narrow} order {order}: grams {} contexts {} n1..n4 {:?} sha {}",
                digest.grams, digest.contexts, digest.count_of_counts, digest.triple_sha256
            );
        }
    }
}

#[test]
fn ngram_count_cuda_max_order_and_short_streams() {
    let tokens = stream(5_000, 0x412_5, 3);
    let counter = NgramCounter::new(0, &tokens).expect("counter");
    for order in [9u32, 10] {
        let (digest, triples, contexts, _) = gpu(&counter, order, 777);
        let (want, want_triples, want_contexts) = oracle_count(&tokens, order).expect("oracle");
        assert_eq!(digest, want);
        assert!(triples == want_triples && contexts == want_contexts);
    }
    let short = [4095u16, 0, 4095];
    let counter = NgramCounter::new(0, &short).expect("counter");
    for order in 0..=4u32 {
        let (digest, _, _, _) = gpu(&counter, order, u64::MAX);
        let (oracle_digest, ..) = oracle_count(&short, order).expect("oracle");
        assert_eq!(digest, oracle_digest, "order {order}");
    }
    assert!(NgramCounter::new(0, &[1, 4096]).is_err());
}

#[test]
fn ngram_count_cuda_sha_is_stable_across_budgets_and_runs() {
    let tokens = stream(1_000_000, 0x412_6, 13);
    let counter = NgramCounter::new(0, &tokens).expect("counter");
    for order in 0..=8u32 {
        let (whole, _, _, one) = gpu(&counter, order, u64::MAX);
        let (again, _, _, _) = gpu(&counter, order, u64::MAX);
        assert_eq!(one, 1);
        assert_eq!(whole, again, "two runs, order {order}");
        for budget in [60_000u64, 9_000] {
            let (split, _, _, partitions) = gpu(&counter, order, budget);
            assert!(partitions > 1, "budget {budget} gave one partition");
            assert_eq!(split, whole, "order {order} budget {budget}");
            eprintln!(
                "stable order {order} budget {budget}: {partitions} partitions, sha {}",
                split.triple_sha256
            );
        }
    }
}

#[test]
fn ngram_count_cuda_sha_covers_a_planted_single_count_change() {
    let tokens = stream(100_000, 0x412_7, 13);
    let counter = NgramCounter::new(0, &tokens).expect("counter");
    let (want, want_triples, _) = oracle_count(&tokens, 3).expect("oracle");
    let plan = counter.plan(3, u64::MAX).expect("plan");
    let mut partition = counter.count_partition(3, &plan[0]).expect("count");
    let at = partition.triples.len() / 2;
    // A consistent plant: the assembler refuses a count whose context record
    // and n1..n4 do not match; this shows the SHA covers the count field.
    plant_one_count(&mut partition, at);
    let mut assembler = NgramAssembler::new(3, true).expect("assembler");
    assembler.push(&partition).expect("push");
    let (digest, triples, _) = assembler.finish();
    assert_ne!(digest.triple_sha256, want.triple_sha256);
    let differing = triples.iter().zip(&want_triples).filter(|(a, b)| a != b).count();
    assert_eq!(differing, 1);
}

/// GPU = oracle on a real token stream, orders 0..5, plus two GPU runs equal.
/// Run explicitly: `UOR_NGRAM_STREAM=path.u16 cargo test ... -- --ignored
/// ngram_count_cuda_real_stream`. Fails when the variable is unset.
#[test]
#[ignore = "needs UOR_NGRAM_STREAM, a UORT token stream of at least 1M tokens"]
fn ngram_count_cuda_real_stream_matches_the_oracle() {
    let path = std::env::var("UOR_NGRAM_STREAM").expect("UOR_NGRAM_STREAM names a UORT stream");
    let tokens = crate::ngram_count::read_uort_tokens(&std::fs::read(&path).expect("read"))
        .expect("UORT tokens");
    assert!(tokens.len() >= 1_000_000, "stream holds {} tokens", tokens.len());
    let counter = NgramCounter::new(0, &tokens).expect("counter");
    for order in 0..=5u32 {
        let (digest, triples, contexts, partitions) = gpu(&counter, order, 4_000_000);
        let (again, _, _, _) = gpu(&counter, order, 4_000_000);
        let (want, want_triples, want_contexts) = oracle_count(&tokens, order).expect("oracle");
        assert_eq!(digest, want, "digest, order {order}");
        assert_eq!(again, digest, "second GPU run, order {order}");
        assert!(triples == want_triples && contexts == want_contexts, "records, order {order}");
        eprintln!(
            "real tokens {} order {order}: partitions {partitions} grams {} contexts {} n1..n4 {:?} gpu=oracle sha {} ctx {}",
            tokens.len(),
            digest.grams,
            digest.contexts,
            digest.count_of_counts,
            digest.triple_sha256,
            digest.context_sha256
        );
    }
}

/// The adjusted counts of `order` under highest order `highest` (claude-418).
fn gpu_adjusted(
    counter: &NgramCounter,
    order: u32,
    highest: u32,
    budget: u64,
) -> (NgramOrderDigest, Vec<NgramTriple>, Vec<NgramContext>, usize) {
    let mut assembler = NgramAssembler::new_adjusted(order, true).expect("assembler");
    let plan = counter.count_order_adjusted(order, highest, budget, &mut assembler).expect("count");
    let (digest, triples, contexts) = assembler.finish();
    (digest, triples, contexts, plan.len())
}

/// Device adjusted counts = the host oracle's, which builds a(g) from sets of
/// left tokens without backing off any triple. Orders 0..3 under N = 3, at one
/// partition and at a budget that forces several. Order 0 sums every order-1
/// partition; orders 1 and 2 pair each range with order k+1 over the same tokens.
#[test]
fn ngram_count_cuda_adjusted_matches_the_oracle() {
    for (len, seed, narrow) in [(100_000usize, 0x418_1u64, 13u32), (1_000_000, 0x418_2, 29)] {
        let tokens = stream(len, seed, narrow);
        assert_eq!(tokens[0], NGRAM_BOS, "rule 4 needs a stream that starts with BOS");
        let counter = NgramCounter::new(0, &tokens).expect("counter");
        for order in 0..=3u32 {
            let (want, want_triples, want_contexts) = oracle_adjusted(&tokens, order, 3).expect("oracle");
            let (_, raw_triples, _) = oracle_count(&tokens, order).expect("raw oracle");
            // Not vacuous: below N the adjusted counts differ from the raw
            // ones; at N they are the raw counts. (Compare the triples: the
            // adjusted SHA sits in its own domain, so it differs even at N.)
            assert_eq!(order < 3, want_triples != raw_triples, "order {order}");
            for budget in [u64::MAX, 9_000] {
                let (digest, triples, contexts, partitions) = gpu_adjusted(&counter, order, 3, budget);
                if budget != u64::MAX {
                    assert!(partitions > 1, "budget {budget} gave one partition, order {order}");
                }
                assert_eq!(digest, want, "digest, len {len} order {order} budget {budget}");
                assert!(triples == want_triples, "triples, len {len} order {order} budget {budget}");
                assert!(contexts == want_contexts, "contexts, len {len} order {order} budget {budget}");
                eprintln!(
                    "adjusted len {len} order {order} budget {budget}: {partitions} partitions, grams {} n1..n4 {:?} sha {}",
                    digest.grams, digest.count_of_counts, digest.triple_sha256
                );
            }
        }
    }
}

#[test]
fn ngram_count_cuda_adjusted_sha_is_stable_across_budgets_and_runs() {
    let tokens = stream(1_000_000, 0x418_3, 13);
    let counter = NgramCounter::new(0, &tokens).expect("counter");
    for highest in [3u32, 8] {
        for order in 0..=highest {
            let (whole, ..) = gpu_adjusted(&counter, order, highest, u64::MAX);
            let (again, ..) = gpu_adjusted(&counter, order, highest, u64::MAX);
            assert_eq!(whole, again, "two runs, order {order} N {highest}");
            for budget in [60_000u64, 9_000] {
                let (split, _, _, partitions) = gpu_adjusted(&counter, order, highest, budget);
                assert!(partitions > 1, "budget {budget} gave one partition");
                assert_eq!(split, whole, "order {order} N {highest} budget {budget}");
            }
        }
    }
}

/// Device adjusted = oracle on a real token stream: orders 0..3 under N = 3 on
/// its first 2M tokens, then the whole stream's adjusted SHA across two budgets
/// and two runs. Run explicitly: `UOR_NGRAM_STREAM=path.u16 cargo test ...
/// -- --ignored ngram_count_cuda_real_stream_adjusted`.
#[test]
#[ignore = "needs UOR_NGRAM_STREAM, a UORT token stream of at least 2M tokens"]
fn ngram_count_cuda_real_stream_adjusted_matches_the_oracle() {
    let path = std::env::var("UOR_NGRAM_STREAM").expect("UOR_NGRAM_STREAM names a UORT stream");
    let tokens = crate::ngram_count::read_uort_tokens(&std::fs::read(&path).expect("read"))
        .expect("UORT tokens");
    assert!(tokens.len() >= 2_000_000, "stream holds {} tokens", tokens.len());
    assert_eq!(tokens[0], NGRAM_BOS, "a UORT stream starts with BOS");
    let slice = &tokens[..2_000_000];
    let counter = NgramCounter::new(0, slice).expect("counter");
    for order in 0..=3u32 {
        let (want, want_triples, want_contexts) = oracle_adjusted(slice, order, 3).expect("oracle");
        let (digest, triples, contexts, partitions) = gpu_adjusted(&counter, order, 3, 1_000_000);
        assert_eq!(digest, want, "slice digest, order {order}");
        assert!(triples == want_triples && contexts == want_contexts, "slice records, order {order}");
        eprintln!(
            "real slice order {order}: partitions {partitions} grams {} n1..n4 {:?} gpu=oracle sha {}",
            digest.grams, digest.count_of_counts, digest.triple_sha256
        );
    }
    let counter = NgramCounter::new(0, &tokens).expect("counter");
    for order in 0..=3u32 {
        let (first, _, _, partitions) = gpu_adjusted(&counter, order, 3, 4_000_000);
        let (again, ..) = gpu_adjusted(&counter, order, 3, 4_000_000);
        let (split, ..) = gpu_adjusted(&counter, order, 3, 1_000_000);
        assert_eq!(again, first, "second GPU run, order {order}");
        assert_eq!(split, first, "budget 1M vs 4M, order {order}");
        eprintln!(
            "real tokens {} order {order}: partitions {partitions} grams {} n1..n4 {:?} sha {} ctx {}",
            tokens.len(),
            first.grams,
            first.count_of_counts,
            first.triple_sha256,
            first.context_sha256
        );
    }
}
