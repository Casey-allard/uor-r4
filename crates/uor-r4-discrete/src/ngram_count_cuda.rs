//! CUDA tests of the exact n-gram counter (claude-412): GPU records equal the
//! CPU oracle's byte for byte, the SHA does not move across partition budgets
//! or runs, and a consistent planted single-count change moves it. Run on wick's 4090
//! with `--release` (the ten-million-token oracle is slow in a debug build).

use crate::ngram_count::{NgramAssembler, NgramOrderDigest, NgramTriple};
use crate::ngram_count_device::NgramCounter;
use crate::ngram_count_oracle::{oracle_count, plant_one_count};

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
            (if narrow == 0 || r % 4 == 0 { r % 4096 } else { r % narrow }) as u16
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
